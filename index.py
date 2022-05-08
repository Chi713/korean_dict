import koreanDict as krdict
import parser as parser
import pprint

f = open(".apikey")
API_KEY = f.read().strip()
f.close()

query = "나무"
sentence = '제 친구 정우가 공항에서 저와 줄리아를 기다리고 있었어요.'
sentence = '어절 내에 포함된 형태소의 리스트를 생성한다'
entry = []

data = parser.parse(sentence)

for i in range(len(data['morpheme'])):
    word = krdict.search(API_KEY,data['morpheme'][i])
    entry.append(word)

print('\nsentence:',sentence)
print('morphemes:',data['morpheme'],'\n')
entry.append(krdict.search(API_KEY,query))
pprint.pprint(entry)
