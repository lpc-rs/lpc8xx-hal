/*
    Set memory sizes to the smallest values found in any of the  LPC800 parts we
    support. This should work correctly even if the memory is actually bigger.
*/
MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 32K
    RAM   : ORIGIN = 0x10000000, LENGTH = 8K
}
