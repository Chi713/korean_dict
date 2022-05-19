from . import search
from . import parser
import pprint
import os
import asyncio
import certifi
import ssl
import cProfile
import pstats

async def main():

    PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(__file__)))

    #fetch apikey
    if "API_KEY" in os.environ:
        API_KEY = os.environ['API_KEY']
    else:
        
        with open(PROJECT_ROOT + "/.apikey") as f:
            API_KEY = f.read().strip("API_KEY=").strip()

    #create complete ssl certification path using intermediate cert
    sslcontext = ssl.create_default_context(cafile = certifi.where())
    sslcontext.load_verify_locations(PROJECT_ROOT + "/certs/krdict.pem")

    krdict = search.Session(API_KEY, sslcontext)
    p = parser.Parser('khaiii')
    #p2 = parser.komoranParse()
    #print(p)
    #print(p2)

    #query = "나무"
    #test_sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
    test_sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
    #test_sentence = '사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'
    entry = []

    data = p.parse(test_sentence)

    entry = []
    word_async = list()

    for morph in data['morpheme']:
        resp = asyncio.create_task(krdict.search(morph))
        word_async.append(resp)

    entry = await asyncio.gather(*word_async)
    pprint.pprint(entry)

if __name__ == '__main__':
    print(os.path.abspath(__file__))
    with cProfile.Profile() as pr:
        asyncio.run(main())
    stats = pstats.Stats(pr)
    stats.sort_stats(pstats.SortKey.TIME)
    stats.print_stats()
