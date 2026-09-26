public final class ConditionalFieldWrites {
    public static boolean staticFlag;
    public boolean instanceFlag;
    public int integerField;

    public static void putStatic(boolean choose) {
        staticFlag = choose ? true : false;
    }

    public void putInstance(boolean choose) {
        instanceFlag = choose ? true : false;
    }

    public void putInteger(boolean choose) {
        integerField = choose ? 2 : 3;
    }

    public static void main(String[] args) {
        ConditionalFieldWrites value = new ConditionalFieldWrites();
        putStatic(false);
        value.putInstance(true);
        System.out.println(staticFlag + ":" + value.instanceFlag);
        putStatic(true);
        value.putInstance(false);
        System.out.println(staticFlag + ":" + value.instanceFlag);
    }
}
