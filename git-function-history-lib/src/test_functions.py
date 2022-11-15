from ctypes.wintypes import PINT

PINT = 1

# def empty_test():
#     print("This is an empty test")

def test_with_assert(n):
    @assert_that("This is a test with an assert")
    def empty_test(t):
        return t == n

        
    assert True

passing_test = test_with_assert
@def_test
def empty_test(n: int) -> list:
    """This is an empty test with a docstring"""
    pass

class TestClass:
    def test_method(self):
        pass

    def test_with_assert(n):
        @assert_that("This is a test with an assert")
        def empty_test(t):
            return t == n
    assert True