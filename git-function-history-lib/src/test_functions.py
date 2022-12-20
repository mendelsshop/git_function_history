from ctypes.wintypes import PINT

PINT = 1

# def empty_test():
#     print("This is an empty test")

def assert_that(message):
    def decorator(func):
        def wrapper(*args, **kwargs):
            print(message)
            return func(*args, **kwargs)
        return wrapper
    return decorator

def test_with_assert(y, n,/,c=7, *, a ,aas = ["a", "b"], **krer,):
    @assert_that("This is a test with an assert")


    
    def empty_test(t):
        return t == n

        
    assert True
class Test:
    pass
passing_test = test_with_assert
@assert_that("This is a test with an assert")

def empty_test(n: int) -> list:
    """This is an empty test with a docstring"""
    pass

# def empty_test(n: int) -> list:

class TestClass:
    def test_method(self):
        pass

    def test_with_assert(n: Test, a =10,* argss,  ** args):
        @assert_that("This is a test with an assert")
        def empty_test(t):
            return t == n
    assert True

