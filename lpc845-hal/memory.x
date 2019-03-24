/*
    Set memory sizes to the smallest values found in any LPC845 parts. This
    should work correctly even if the memory is actually bigger.
*/
MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 64K
    RAM   : ORIGIN = 0x10000000, LENGTH = 16K
}
