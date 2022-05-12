import koreanDict
import parser
import pprint
import os
import asyncio
import time
import certifi
import ssl

#fetch apikey
if "API_KEY" in os.environ:
    API_KEY = os.environ['API_KEY']
else:
    f = open(".apikey")
    API_KEY = f.read().strip("API_KEY=").strip()
    f.close()

#create complete ssl certification path using intermediate cert
sslcontext = ssl.create_default_context(cafile = certifi.where())
sslcontext.load_verify_locations("./certs/krdict.pem")

krdict = koreanDict.Session(API_KEY, sslcontext)
parser = parser.Parser()

#query = "나무"
#test_sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
test_sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
#test_sentence = '사람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'
entry = []

data = parser.parse(test_sentence)

async def main(data):
    entry = []
    word_async = list()

    for morph in data['morpheme']:
        resp = asyncio.create_task(krdict.search(morph))
        word_async.append(resp)

    entry = await asyncio.gather(*word_async)
    pprint.pprint(entry)

tic = time.perf_counter()
asyncio.run(main(data))
toc = time.perf_counter()

timePass = toc-tic
print(timePass)
