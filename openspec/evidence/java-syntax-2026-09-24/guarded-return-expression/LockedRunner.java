public final class LockedRunner {
    public static void main(String[] args) {
        Locked value = new Locked();
        value.n = 7;
        System.out.println("locked:" + value.locked());
        value.n = -4;
        System.out.println("locked:" + value.locked());
    }
}
