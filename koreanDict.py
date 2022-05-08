import requests
import certifi
import xml.etree.ElementTree as ET
import time

URL = "https://krdict.korean.go.kr/api/search?"

def search(key,query):
    PARAMS = {'key': key,
        'q': query,
        'translated': 'y',
        'trans_lang': '1'}

    tic = time.perf_counter()
    r = requests.get(url = URL, params = PARAMS, verify=certifi.where())
    toc = time.perf_counter()

    if r.status_code != 200:
        print("error in fetching data from krdict.korean.go.kr/api")
        return r.raise_for_status

    data = r.content.decode('UTF-8').strip()
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
        
    timePass = toc-tic
    print("time for request:",timePass)

    return entry 
