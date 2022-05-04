import requests
import certifi
import xml.etree.ElementTree as ET

URL = "https://krdict.korean.go.kr/api/search?"

#f = open(".apikey")
#API_KEY = f.read().strip()
#f.close()

#theQuery = "나무"

def search(key,query):
    PARAMS = {'key': key,
        'q': query,
        'translated': 'y',
        'trans_lang': '1'}

    r = requests.get(url = URL, params = PARAMS, verify=certifi.where())
    data = r.content.decode('UTF-8').strip()
    root = ET.fromstring(data)
    word = root[7][1].text
    defi = []
    expl = []

    for child in root[7]:
        if child.tag == 'sense':
            defi.append(child[2][1].text)
            expl.append(child[2][2].text)

    #print(word)
    #for i in range(len(defi)):
        #print(defi[i],expl[i])
        
    entry = {'word': word,
        'definition': defi,
        'explaination': expl}
    return entry
