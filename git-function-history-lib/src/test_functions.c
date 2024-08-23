#include <stdio.h>

void test_function(void);

static void test_function2(void)
{
    printf("Hello World!");

    // printf("Hello World!" );
}

int main()
{
    printf("Hello World!");
    test_function();
    test_function2();
    // test_functions();
    // test_functions2();
    return 0;
}

void test_function(void)
{
    printf("Hello World!");
}

int empty_test()
{f}

