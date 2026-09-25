public class DefaultMiddle {
    public static int choose(int value) {
        int result = 0;
        switch (value) {
            case 9:
                result += 90;
            default:
                result += 2;
            case 1:
                result += 1;
                break;
            case 4:
                result += 4;
                break;
        }
        return result;
    }
}
