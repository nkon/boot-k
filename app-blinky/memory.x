MEMORY {
    IMAGE_HEADER : ORIGIN = 0x10020000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10020100, LENGTH = 0xe0000 - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS {
    /* ### Boot loader */
    .image_header ORIGIN(IMAGE_HEADER) :
    {
        KEEP(*(.image_header));
    } > IMAGE_HEADER
} INSERT BEFORE .text;
