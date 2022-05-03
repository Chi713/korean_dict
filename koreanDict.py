import requests
import certifi

URL = "https://krdict.korean.go.kr/api/search?"

f = open(".apikey")
API_KEY = f.read().strip()
f.close()
query = "나무"

#print(API_KEY)
PARAMS = {'key': API_KEY,
    'q': query,
    'translated': 'y',
    'trans_lang': '1'}

r = requests.get(url = URL, params = PARAMS, verify=certifi.where())

#r = http.request('GET', URL)
#print(r.data)
print(r.content)
