#import certifi
import xml.etree.ElementTree as ET
#import ssl
import aiohttp

class Session:
    BASE_URL = "https://krdict.korean.go.kr/api/search?"

    def __init__(self, key, sslcontext):
        self.key = key
        self.sslcontext = sslcontext

    async def search(self, query):

        PARAMS = {'key': self.key,
            'q': query,
            'translated': 'y',
            'trans_lang': '1'} 

        #GET request from krdict.go.kr API
        async def fetch(session, params):
            async with session.get(self.BASE_URL, params=params, ssl=self.sslcontext) as r:
                return await r.read()

        async def parse(data):
            root = ET.fromstring(data)
            #word = query
            defi = []
            expl = []

            for branch in [x for x in root[7:] if x[1].text == query]:
                for middle in [x for x in branch if x.tag == 'sense']:
                    for child in [x for x in middle if x.tag == 'translation']:
                        #word = branch[1].text
                        defi.append(str(child[1].text).strip())
                        expl.append(str(child[2].text).strip())

            return {'word': query,
                'definition': defi,
                'explaination': expl,}

        async with aiohttp.ClientSession() as session:
            data = await fetch(session, PARAMS)
            return await parse(data)

    #if r.status_code != 200:
        #print("error in fetching data from krdict.korean.go.kr/api")
        #return r.raise_for_status
