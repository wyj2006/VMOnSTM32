#define PG7 (*(char *)(1024 * 100 + 1024 * 1024 + 0))

int main() {
  while (1) {
    PG7 = 1 - PG7;
    for (int i = 0; i < 10; i++)
      ;
  }
}