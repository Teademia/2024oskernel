#include "stdio.h"
#include "stdlib.h"
#include "unistd.h"

static char buffer[30];
void test_chdir(void)
{
    TEST_START(__func__);
    int ret = chdir("test_chdir");
    printf("chdir ret: %d\n", ret);
    assert(ret == 0);
    getcwd(buffer, 30);
    printf("  current working dir : %s\n", buffer);
    TEST_END(__func__);
}

int main(void)
{
    test_chdir();
    return 0;
}