public class SwitchFallthroughOrder {
    private static int calls;

    private static int selector(int value) {
        calls++;
        return value;
    }

    public static int choose(int value) {
        int result = 0;
        switch (selector(value)) {
            case 9:
                result += 90;
            case 1:
                result += 1;
                break;
            case 4:
                result += 4;
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
