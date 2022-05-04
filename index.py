from khaiii import KhaiiiApi
import koreanDict as krdict

#api = KhaiiiApi()
#for word in api.analyze('안녕, 세상.'):
#    print(word)

f = open(".apikey")
API_KEY = f.read().strip()
f.close()

query = "나무"

entry = krdict.search(API_KEY,query)
print(entry)
