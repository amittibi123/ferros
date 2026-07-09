#ifndef FERROS_H
#define FERROS_H

#include <stdint.h>

#define SYS_PRINT 0
#define SYS_READ  1

// מבנה ה-String החכם של Ferros
typedef struct {
    const char *data;
    uint64_t len;
} FString;

#define FSTR_CONSTANT(s) { .data = s, .len = (sizeof(s) - 1) }

// פונקציית עזר פנימית לחישוב אורך של string literal רגיל ב-C
static inline uint64_t fstrlen(const char *s) {
    uint64_t len = 0;
    while (s[len] != '\0') len++;
    return len;
}

// מקרו/פונקציה מהירה ליצירת FString מסטרינג קבוע
static inline FString fstr(const char *s) {
    FString str;
    str.data = s;
    str.len = fstrlen(s);
    return str;
}

// פונקציה לבדיקה אם FString שווה למחרוזת C רגילה (Null-terminated)
static inline int fstr_eq(FString fstr, const char *c_str) {
    uint64_t i = 0;

    // רצים על המחרוזת
    while (i < fstr.len) {
        // אם ה-FString עדיין לא נגמרה, אבל מחרוזת ה-C כבר הגיעה ל-0, הן לא שוות
        if (c_str[i] == '\0') {
            return 0;
        }

        // אם יש אי-התאמה בתווים
        if (fstr.data[i] != c_str[i]) {
            return 0;
        }

        i++;
    }

    // אחרי שסיימנו את כל אורך ה-FString, מחרוזת ה-C חייבת בדיוק להסתיים
    return c_str[i] == '\0';
}

static inline FString fstring_first_word(FString str) {
    uint64_t new_len = 0;

    while (new_len < str.len) {
        char c = str.data[new_len];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            break;
        }
        new_len++;
    }

    FString first_word;
    first_word.data = str.data;
    first_word.len = new_len;
    return first_word;
}

static inline FString fstring_after_first_word(FString str) {
    uint64_t i = 0;

    // 1. שלב ראשון: מדלגים על המילה הראשונה (עד שמוצאים רווח או סוף מחרוזת)
    while (i < str.len) {
        char c = str.data[i];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            break; // הגענו לסוף המילה הראשונה
        }
        i++;
    }

    // 2. שלב שני: מדלגים על כל הרווחים שבין המילה הראשונה לשנייה (למשל אם הקלידו "help    clear")
    while (i < str.len) {
        char c = str.data[i];
        if (c != ' ' && c != '\t' && c != '\n' && c != '\r') {
            break; // מצאנו את התו הראשון של המילה השנייה!
        }
        i++;
    }

    // 3. שלב שלישי: בונים את ה-FString של מה שנשאר
    FString after_word;

    // המצביע זז קדימה בזיכרון בדיוק לנקודה שבה המילה השנייה מתחילה
    after_word.data = &str.data[i];

    // האורך החדש הוא האורך המקורי פחות כמות התווים שדילגנו עליהם
    after_word.len = str.len - i;

    return after_word;
}

// פונקציית עזר פנימית להמרת מספר לטקסט
static inline char *itoa_simple(uint64_t value, char *result, int base) {
    if (base < 2 || base > 16) return result;
    char *ptr = result, *ptr1 = result, tmp_char;
    uint64_t tmp_value = value;

    if (value == 0) {
        *ptr++ = '0';
        *ptr = '\0';
        return result;
    }

    while (tmp_value) {
        int rem = tmp_value % base;
        *ptr++ = (rem < 10) ? (rem + '0') : (rem - 10 + 'a');
        tmp_value /= base;
    }
    *ptr = '\0';

    ptr--;
    while (ptr1 < ptr) {
        tmp_char = *ptr;
        *ptr = *ptr1;
        *ptr1 = tmp_char;
        ptr--;
        ptr1++;
    }
    return result;
}

// פונקציית ה-printf המאוחדת - תומכת עכשיו גם ב-%s עבור FString וגם ב-%c עבור C-String רגיל
static inline int printf(const char *format, ...) {
    char buffer[1024];
    char *buf_ptr = buffer;

    __builtin_va_list args;
    __builtin_va_start(args, format);

    for (const char *p = format; *p != '\0'; p++) {
        if (*p != '%') {
            *buf_ptr++ = *p;
            continue;
        }

        p++;

        switch (*p) {
            case 's': {
                // הדפסת FString חכם
                FString s = __builtin_va_arg(args, FString);
                for (uint64_t i = 0; i < s.len; i++) {
                    *buf_ptr++ = s.data[i];
                }
                break;
            }
            case 'c': {
                // תמיכה לאחור בסטרינג C רגיל (מצביע null-terminated)
                char *s = __builtin_va_arg(args, char *);
                while (*s) *buf_ptr++ = *s++;
                break;
            }
            case 'd': {
                uint64_t d = __builtin_va_arg(args, uint64_t);
                char num_buf[32];
                itoa_simple(d, num_buf, 10);
                char *n = num_buf;
                while (*n) *buf_ptr++ = *n++;
                break;
            }
            case 'x': {
                uint64_t x = __builtin_va_arg(args, uint64_t);
                char num_buf[32];
                itoa_simple(x, num_buf, 16);
                char *n = num_buf;
                while (*n) *buf_ptr++ = *n++;
                break;
            }
            case '%': {
                *buf_ptr++ = '%';
                break;
            }
            default:
                *buf_ptr++ = *p;
                break;
        }
    }

    __builtin_va_end(args);
    *buf_ptr = '\0';

    uint64_t len = buf_ptr - buffer;

    int64_t ret;
    asm volatile (
        "syscall"
        : "=a"(ret)
        : "a"(SYS_PRINT), "S"(buffer), "d"(len)
        : "rcx", "r11", "memory"
    );

    return ret;
}

// ה-Syscall הגולמי שעובד לך מצוין
static inline uint64_t ferros_read(char *buffer, uint64_t max_len) {
    uint64_t bytes_read;
    asm volatile (
        "syscall"
        : "=a"(bytes_read)
        : "a"(SYS_READ), "S"(buffer), "d"(max_len)
        : "rcx", "r11", "memory"
    );
    return bytes_read;
}

// פונקציית הקלט הגבוהה - בונה את ה-FString בצורה בטוחה ישירות מהבאפר
static inline FString input(char *buffer, uint64_t max_len) {
    uint64_t len = ferros_read(buffer, max_len);

    FString str;
    str.data = buffer;
    str.len = len;
    return str;
}

#endif // FERROS_H
