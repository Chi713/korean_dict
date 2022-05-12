import koreanDict as krdict
import parser as parser
import pprint
import os
import asyncio
import time

#fetch apikey
if "API_KEY" in os.environ:
    API_KEY = os.environ['API_KEY']
else:
    f = open(".apikey")
    API_KEY = f.read().strip("API_KEY=").strip()
    f.close()
    #print(API_KEY)

#append intermediate cert to certifi


query = "나무"
#sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
#sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
sentence = '람의 말이나 행동, 성격에서 잘못된 점이나 부족한 점을 지적하여 말하다.'
entry = []

data = parser.parse(sentence)

async def main(data):
    entry = []
    word_async = list()

    for morph in data['morpheme']:
        resp = asyncio.create_task(krdict.search(API_KEY, morph))
        word_async.append(resp)

    entry = await asyncio.gather(*word_async)
    pprint.pprint(entry)

tic = time.perf_counter()
asyncio.run(main(data))
toc = time.perf_counter()

timePass = toc-tic
print(timePass)
