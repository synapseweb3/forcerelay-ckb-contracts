// https://github.com/xxuejie/lib-dummy-atomics/blob/50dc5fefb215bc93e761fb655d7a4fdade04c2d1/atomics.c

// MIT License
//
// Copyright (c) 2023 Xuejie Xiao
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#include <stdbool.h>
#include <stdint.h>

#define _ATOMIC_EXCHANGE_IMPL(t)                                               \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst = val;                                                                  \
  return old

uint8_t __atomic_exchange_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_EXCHANGE_IMPL(uint8_t);
}

uint16_t __atomic_exchange_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_EXCHANGE_IMPL(uint16_t);
}

uint32_t __atomic_exchange_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_EXCHANGE_IMPL(uint32_t);
}

uint64_t __atomic_exchange_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_EXCHANGE_IMPL(uint64_t);
}

#undef _ATOMIC_EXCHANGE_IMPL

#define _ATOMIC_COMPARE_IMPL(t)                                                \
  (void)weak;                                                                  \
  (void)success_memorder;                                                      \
  (void)failure_memorder;                                                      \
  t *dst = (t *)ptr;                                                           \
  t *old = (t *)expected;                                                      \
  if (*dst == *old) {                                                          \
    *dst = desired;                                                            \
    return true;                                                               \
  } else {                                                                     \
    *old = *dst;                                                               \
    return false;                                                              \
  }

bool __atomic_compare_exchange_1(volatile void *ptr, void *expected,
                                 uint8_t desired, bool weak,
                                 int success_memorder, int failure_memorder) {
  _ATOMIC_COMPARE_IMPL(uint8_t);
}

bool __atomic_compare_exchange_2(volatile void *ptr, void *expected,
                                 uint16_t desired, bool weak,
                                 int success_memorder, int failure_memorder) {
  _ATOMIC_COMPARE_IMPL(uint16_t);
}

bool __atomic_compare_exchange_4(volatile void *ptr, void *expected,
                                 uint32_t desired, bool weak,
                                 int success_memorder, int failure_memorder) {
  _ATOMIC_COMPARE_IMPL(uint32_t);
}

bool __atomic_compare_exchange_8(volatile void *ptr, void *expected,
                                 uint64_t desired, bool weak,
                                 int success_memorder, int failure_memorder) {
  _ATOMIC_COMPARE_IMPL(uint64_t);
}

#undef _ATOMIC_COMPARE_IMPL

#define _ATOMIC_LOAD_IMPL(t)                                                   \
  (void)memorder;                                                              \
  return *((t *)ptr)

uint8_t __atomic_load_1(const volatile void *ptr, int memorder) {
  _ATOMIC_LOAD_IMPL(uint8_t);
}

uint16_t __atomic_load_2(const volatile void *ptr, int memorder) {
  _ATOMIC_LOAD_IMPL(uint16_t);
}

uint32_t __atomic_load_4(const volatile void *ptr, int memorder) {
  _ATOMIC_LOAD_IMPL(uint32_t);
}

uint64_t __atomic_load_8(const volatile void *ptr, int memorder) {
  _ATOMIC_LOAD_IMPL(uint64_t);
}

#undef _ATOMIC_LOAD_IMPL

#define _ATOMIC_STORE_IMPL(t)                                                  \
  (void)memorder;                                                              \
  *((t *)ptr) = val

void __atomic_store_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_STORE_IMPL(uint8_t);
}

void __atomic_store_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_STORE_IMPL(uint16_t);
}

void __atomic_store_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_STORE_IMPL(uint32_t);
}

void __atomic_store_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_STORE_IMPL(uint64_t);
}

#undef _ATOMIC_STORE_IMPL

#define _ATOMIC_FETCH_ADD_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst += val;                                                                 \
  return old

uint8_t __atomic_fetch_add_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_ADD_IMPL(uint8_t);
}

uint16_t __atomic_fetch_add_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_ADD_IMPL(uint16_t);
}

uint32_t __atomic_fetch_add_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_ADD_IMPL(uint32_t);
}

uint64_t __atomic_fetch_add_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_ADD_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_ADD_IMPL

#define _ATOMIC_FETCH_SUB_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst -= val;                                                                 \
  return old

uint8_t __atomic_fetch_sub_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_SUB_IMPL(uint8_t);
}

uint16_t __atomic_fetch_sub_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_SUB_IMPL(uint16_t);
}

uint32_t __atomic_fetch_sub_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_SUB_IMPL(uint32_t);
}

uint64_t __atomic_fetch_sub_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_SUB_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_SUB_IMPL

#define _ATOMIC_FETCH_AND_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst &= val;                                                                 \
  return old

uint8_t __atomic_fetch_and_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_AND_IMPL(uint8_t);
}

uint16_t __atomic_fetch_and_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_AND_IMPL(uint16_t);
}

uint32_t __atomic_fetch_and_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_AND_IMPL(uint32_t);
}

uint64_t __atomic_fetch_and_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_AND_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_AND_IMPL

#define _ATOMIC_FETCH_XOR_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst ^= val;                                                                 \
  return old

uint8_t __atomic_fetch_xor_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_XOR_IMPL(uint8_t);
}

uint16_t __atomic_fetch_xor_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_XOR_IMPL(uint16_t);
}

uint32_t __atomic_fetch_xor_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_XOR_IMPL(uint32_t);
}

uint64_t __atomic_fetch_xor_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_XOR_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_XOR_IMPL

#define _ATOMIC_FETCH_OR_IMPL(t)                                               \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst |= val;                                                                 \
  return old

uint8_t __atomic_fetch_or_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_OR_IMPL(uint8_t);
}

uint16_t __atomic_fetch_or_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_OR_IMPL(uint16_t);
}

uint32_t __atomic_fetch_or_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_OR_IMPL(uint32_t);
}

uint64_t __atomic_fetch_or_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_OR_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_OR_IMPL

#define _ATOMIC_FETCH_NAND_IMPL(t)                                             \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  t old = *dst;                                                                \
  *dst = ~(*dst & val);                                                        \
  return old

uint8_t __atomic_fetch_nand_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_FETCH_NAND_IMPL(uint8_t);
}

uint16_t __atomic_fetch_nand_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_FETCH_NAND_IMPL(uint16_t);
}

uint32_t __atomic_fetch_nand_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_FETCH_NAND_IMPL(uint32_t);
}

uint64_t __atomic_fetch_nand_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_FETCH_NAND_IMPL(uint64_t);
}

#undef _ATOMIC_FETCH_NAND_IMPL

#define _ATOMIC_ADD_FETCH_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst += val;                                                                 \
  return *dst

uint8_t __atomic_add_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_ADD_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_add_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_ADD_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_add_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_ADD_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_add_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_ADD_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_ADD_FETCH_IMPL

#define _ATOMIC_SUB_FETCH_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst -= val;                                                                 \
  return *dst

uint8_t __atomic_sub_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_SUB_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_sub_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_SUB_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_sub_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_SUB_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_sub_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_SUB_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_SUB_FETCH_IMPL

#define _ATOMIC_AND_FETCH_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst &= val;                                                                 \
  return *dst

uint8_t __atomic_and_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_AND_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_and_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_AND_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_and_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_AND_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_and_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_AND_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_AND_FETCH_IMPL

#define _ATOMIC_XOR_FETCH_IMPL(t)                                              \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst ^= val;                                                                 \
  return *dst

uint8_t __atomic_xor_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_XOR_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_xor_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_XOR_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_xor_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_XOR_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_xor_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_XOR_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_XOR_FETCH_IMPL

#define _ATOMIC_OR_FETCH_IMPL(t)                                               \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst |= val;                                                                 \
  return *dst

uint8_t __atomic_or_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_OR_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_or_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_OR_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_or_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_OR_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_or_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_OR_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_OR_FETCH_IMPL

#define _ATOMIC_NAND_FETCH_IMPL(t)                                             \
  (void)memorder;                                                              \
  t *dst = (t *)ptr;                                                           \
  *dst = ~((*dst) & val);                                                      \
  return *dst

uint8_t __atomic_nand_fetch_1(volatile void *ptr, uint8_t val, int memorder) {
  _ATOMIC_NAND_FETCH_IMPL(uint8_t);
}

uint16_t __atomic_nand_fetch_2(volatile void *ptr, uint16_t val, int memorder) {
  _ATOMIC_NAND_FETCH_IMPL(uint16_t);
}

uint32_t __atomic_nand_fetch_4(volatile void *ptr, uint32_t val, int memorder) {
  _ATOMIC_NAND_FETCH_IMPL(uint32_t);
}

uint64_t __atomic_nand_fetch_8(volatile void *ptr, uint64_t val, int memorder) {
  _ATOMIC_NAND_FETCH_IMPL(uint64_t);
}

#undef _ATOMIC_NAND_FETCH_IMPL
