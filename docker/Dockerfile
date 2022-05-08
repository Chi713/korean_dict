FROM pytorch/pytorch:latest

RUN apt-get update && apt-get install -y build-essential git
RUN git clone https://github.com/Chi713/korean_dict_py.git
RUN git clone https://github.com/kakao/khaiii.git
WORKDIR /workspace/khaiii

RUN pip install cython
RUN pip install --upgrade pip
RUN pip install -r requirements.txt

RUN mkdir build
WORKDIR /workspace/khaiii/build

RUN cmake ..
RUN make all
RUN make resource
WORKDIR /workspace/khaiii/build/package_python/
RUN ls /workspace/khaiii/build/package_python/
#RUN pip install /workspace/khaiii/build/package_python/

RUN apt-get update -y
RUN apt-get install -y language-pack-ko
RUN locale-gen en_US.UTF-8
RUN update-locale LANG=en_US.UTF-8

RUN pip install requests certifi prettyprint
WORKDIR /workspace/korean_dict_py
RUN touch .apikey
RUN echo "2ED68630E3AE8569BD8997513415907C" >> .apikey
CMD ["python", "/workspace/korean_dict_py/index.py"]
#RUN python index.py

