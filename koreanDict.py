#import requests
import certifi
import xml.etree.ElementTree as ET
#import time
import ssl
#import asyncio
import aiohttp

#query = "나무"

#f = open(".apikey")
#API_KEY = f.read().strip("API_KEY=").strip()
#f.close()

BASE_URL = "https://krdict.korean.go.kr/api/search?"

sslcontext = ssl.create_default_context(
    cafile = certifi.where())

async def fetch(session, params):
    global BASE_URL
    global sslcontext
    async with session.get(BASE_URL, params=params, ssl=sslcontext) as r:
        return await r.read()

async def search(key,query):

    PARAMS = {'key': key,
        'q': query,
        'translated': 'y',
        'trans_lang': '1'}
    
    async with aiohttp.ClientSession() as session:
        data = await fetch(session, PARAMS)


    #tic = time.perf_counter()
    #r = requests.get(url = BASE_URL, params = PARAMS, verify=certifi.where())
    #toc = time.perf_counter()

    #if r.status_code != 200:
        #print("error in fetching data from krdict.korean.go.kr/api")
        #return r.raise_for_status

        #data = str(r.read()).decode('UTF-8').strip()
        root = ET.fromstring(data)
        word = ""
        defi = []
        expl = []
        flag = True

        for branch in root[7:]:
            for middle in [x for x in branch if x.tag == 'sense' and flag]:
                for child in [x for x in middle if x.tag == 'translation']:
                    word = branch[1].text
                    defi.append(str(child[1].text).strip())
                    expl.append(str(child[2].text).strip())
                    flag = False

        entry = {'word': word,
            'definition': defi,
            'explaination': expl}
        
        #print(entry)

    #timePass = toc-tic
    #print("time for request:",timePass)

#asyncio.run(search(API_KEY, query))

        #return entry 
