from ctypes.wintypes import PINT

PINT = 1

# def empty_test():
#     print("This is an empty test")

def test_with_assert(y, n,/,c=7, *, a , args, **kwargs):
    @assert_that("This is a test with an assert")
    def empty_test(t):
        return t == n

        
    assert True
class Test:
    pass
passing_test = test_with_assert
@def_test
def empty_test(n: int) -> list:
    """This is an empty test with a docstring"""
    pass

class TestClass:
    def test_method(self):
        pass

    def test_with_assert(n: Test):
        @assert_that("This is a test with an assert")
        def empty_test(t):
            return t == n
    assert True

