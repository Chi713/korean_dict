FROM pytorch/pytorch:latest

RUN apt-get update && apt-get install -y build-essential git ca-certificates libssl-dev libffi-dev gnupg
RUN apt-get install -y curl openssl 
RUN git clone https://github.com/Chi713/korean_dict_py.git
RUN git clone https://github.com/kakao/khaiii.git
WORKDIR /workspace/khaiii

RUN pip install cython
RUN pip install --upgrade pip
RUN pip install -r requirements.txt

RUN mkdir build
WORKDIR /workspace/khaiii/build

#make and install khaiii
RUN cmake ..
RUN make all
RUN make resource
RUN make install
RUN make package_python
WORKDIR /workspace/khaiii/build/package_python/
RUN pip install /workspace/khaiii/build/package_python/

RUN apt-get update -y
RUN apt-get install -y language-pack-ko
RUN locale-gen en_US.UTF-8
RUN update-locale LANG=en_US.UTF-8

RUN pip install requests certifi prettyprint
WORKDIR /workspace/korean_dict_py
RUN touch .apikey

RUN cat certs/krdict.pem >> /opt/conda/lib/python3.8/site-packages/certifi/cacert.pem
RUN cp /opt/conda/lib/python3.8/site-packages/certifi/cacert.pem certs/
#CMD ["python", "/workspace/korean_dict_py/testSSL.py"]
CMD ["python", "/workspace/korean_dict_py/index.py"]
#CMD ["ls","/workspace/korean_dict_py"]
