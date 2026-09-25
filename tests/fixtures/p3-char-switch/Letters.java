public class Letters {
    static int letter(char c) {
        switch (c) {
            case 'a':
                return 1;
            case 'b':
                return 2;
            default:
                return 0;
        }
    }

    static int number(int n) {
        switch (n) {
            case 97:
                return 1;
            default:
                return 0;
        }
    }
}
