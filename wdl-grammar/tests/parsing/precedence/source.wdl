# This is a test of operator precedence

version 1.1

task test {
    Boolean a = 0 || 1 && 1 == 0 != 1 < 0 <= 1 > 0 >= 1 + 2 - 3 * 4 / 5 % 6
    Integer b = (1 + 2) - (3 * 4) / (5 % 6)
}
