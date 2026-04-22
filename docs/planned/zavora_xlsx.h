/* zavora_xlsx.h — C FFI header for zavora-xlsx
 *
 * Generated for the `cffi` feature flag.
 * Link with: -lzavora_xlsx
 *
 * Usage:
 *   ZavoraWorkbook* wb = zavora_workbook_new();
 *   zavora_worksheet_write_string(wb, 0, 0, 0, "Hello");
 *   zavora_worksheet_write_number(wb, 0, 0, 1, 42.5);
 *   zavora_workbook_save(wb, "output.xlsx");
 *   zavora_workbook_free(wb);
 */

#ifndef ZAVORA_XLSX_H
#define ZAVORA_XLSX_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle to a Workbook. */
typedef struct ZavoraWorkbook ZavoraWorkbook;

/* Create a new empty workbook. Returns NULL on failure. */
ZavoraWorkbook* zavora_workbook_new(void);

/* Write a string value to a cell. Returns 0 on success, -1 on failure. */
int zavora_worksheet_write_string(ZavoraWorkbook* wb, uint32_t sheet,
                                   uint32_t row, uint16_t col,
                                   const char* text);

/* Write a numeric value to a cell. Returns 0 on success, -1 on failure. */
int zavora_worksheet_write_number(ZavoraWorkbook* wb, uint32_t sheet,
                                   uint32_t row, uint16_t col,
                                   double value);

/* Write a boolean value to a cell. Returns 0 on success, -1 on failure. */
int zavora_worksheet_write_bool(ZavoraWorkbook* wb, uint32_t sheet,
                                 uint32_t row, uint16_t col,
                                 int value);

/* Save the workbook to a file. Returns 0 on success, -1 on failure. */
int zavora_workbook_save(ZavoraWorkbook* wb, const char* path);

/* Save the workbook to an in-memory buffer.
 * On success, *out_ptr and *out_len are set. Free with zavora_buffer_free().
 * Returns 0 on success, -1 on failure. */
int zavora_workbook_save_to_buffer(ZavoraWorkbook* wb,
                                    uint8_t** out_ptr, size_t* out_len);

/* Free a buffer allocated by zavora_workbook_save_to_buffer. */
void zavora_buffer_free(uint8_t* ptr, size_t len);

/* Free a workbook handle. */
void zavora_workbook_free(ZavoraWorkbook* wb);

/* Get the last error message. Returns NULL if no error.
 * The string is valid until the next FFI call. */
const char* zavora_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* ZAVORA_XLSX_H */
