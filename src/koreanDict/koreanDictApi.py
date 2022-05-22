import pprint
import os
import asyncio
import certifi
import ssl
from pathlib import Path
#import time
import cProfile
import pstats

from . import search
from . import parser
from . import csvParser

async def main(tsv_file: str):

    PROJECT_ROOT = Path(__file__).parents[2]

    pp= pprint.PrettyPrinter(compact=True)

    #fetch apikey from file or env
    if "API_KEY" in os.environ:
        API_KEY = os.environ['API_KEY']
    else:
        API_KEY_PATH = PROJECT_ROOT / '.apikey' 
        with API_KEY_PATH.open() as f:
            API_KEY = f.read().strip("API_KEY=").strip()

    #create complete ssl certification path using intermediate cert
    sslcontext = ssl.create_default_context(cafile = certifi.where())
    sslcontext.load_verify_locations((PROJECT_ROOT / "certs/krdict.pem").resolve())


    cp = csvParser.csvParse()
    data = cp.parse(PROJECT_ROOT / tsv_file)
    sentence = data[1][4]
    
    entry = list()
    word = list()

    krdict = search.Session(API_KEY, sslcontext)
    p = parser.Parser('khaiii')

    data = p.parse(sentence)
    for morph in data['morpheme']:
        resp = asyncio.create_task(krdict.search(morph))
        word.append(resp)

    entry = await asyncio.gather(*word)
    pp.pprint(entry)

if __name__ == '__main__':
    with cProfile.Profile() as pr:
        #query = "나무"
        #test_sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
        #test_sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
        #test_sentence = '사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'


        PROJECT_ROOT = Path(__file__).parents[2]

        cp = csvParser.csvParse()
        data = cp.parse(PROJECT_ROOT / "resources/bite_sisters.tsv")
        test_sentence = data[1][4]

        asyncio.run(main(test_sentence))
    stats = pstats.Stats(pr)
    stats.sort_stats(pstats.SortKey.TIME)
    stats.print_stats()
