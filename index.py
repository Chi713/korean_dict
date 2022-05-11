import koreanDict as krdict
import parser as parser
import pprint
import os
import asyncio
import time

if "API_KEY" in os.environ:
    API_KEY = os.environ['API_KEY']
else:
    f = open(".apikey")
    API_KEY = f.read().strip("API_KEY=").strip()
    f.close()
    #print(API_KEY)

query = "나무"
#sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
entry = []

data = parser.parse(sentence)

async def main():
    #entry.append(await krdict.search(API_KEY, query))
    for (morph, tag) in zip(data['morpheme'], data['tag']):
        word = await krdict.search(API_KEY,morph)
        global entry
        entry.append(word)
        print(word)

    #print('\nsentence:',sentence)
    #print('morphemes:',data['morpheme'],'\n')
    #entry.append(krdict.search(API_KEY,query))
    pprint.pprint(entry)
    pprint.pprint(data['tag'])
tic = time.perf_counter()
asyncio.run(main())
toc = time.perf_counter()

timePass = toc-tic
print(timePass)
