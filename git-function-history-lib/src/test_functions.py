from ctypes.wintypes import PINT


PINT = 1

def empty_test():
    print("This is an empty test")

def test_with_assert():
    assert True

passing_test = test_with_assert

def empty_test():
    """This is an empty test with a docstring"""
    pass
