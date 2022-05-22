#import certifi
import xml.etree.ElementTree as ET
import ssl
import aiohttp
from dataclasses import dataclass

@dataclass(frozen=True, order=True)
class dict_entry:
    word: str
    definition: list
    explaination: list

class Session:
    BASE_URL = "https://krdict.korean.go.kr/api/search?"

    def __init__(self, key: str, sslcontext: ssl.SSLContext):
        self.key = key
        self.sslcontext = sslcontext

    async def search(self, query: str):

        PARAMS = {'key': self.key,
            'q': query,
            'translated': 'y',
            'trans_lang': '1'} 

        #GET request from krdict.go.kr API
        async def fetch(session: aiohttp.client.ClientSession, params: dict):
            async with session.get(self.BASE_URL, params=params, ssl=self.sslcontext) as r:
                data = await r.read()
                return data.decode('UTF-8')

        async def parse(data: str):
            root = ET.fromstring(data)
            defi = []
            expl = []

            for branch in [x for x in root[7:] if x[1].text == query]:
                for middle in [x for x in branch if x.tag == 'sense']:
                    for child in [x for x in middle if x.tag == 'translation']:
                        defi.append(str(child[1].text).strip())
                        expl.append(str(child[2].text).strip())

            return dict_entry(query, defi, expl)

            #return {'word': query,
                #'definition': defi,
                #'explaination': expl,}

        async with aiohttp.ClientSession() as session:
            data = await fetch(session, PARAMS)
            return await parse(data)

    #if r.status_code != 200:
        #print("error in fetching data from krdict.korean.go.kr/api")
        #return r.raise_for_status
