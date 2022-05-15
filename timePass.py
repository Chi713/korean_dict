import time

def timer():
    def wrapper(func):
        tic = time.perf_counter()
        func()
        toc = time.perf_counter()
        print(toc-tic)
    return wrapper
