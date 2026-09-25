public class TableswitchFallthrough {
    private static int calls;

    private static int selector(int value) {
        calls++;
        return value;
    }

    public static int choose(int value) {
        int result = 0;
        switch (selector(value)) {
            case 4:
                result += 40;
            case 1:
            case 2:
                result += 1;
                break;
            case 3:
                result += 3;
                break;
            default:
                result -= 1;
        }
        return result;
    }

    public static int calls() {
        return calls;
    }
}
