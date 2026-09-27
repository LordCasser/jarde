package dt26;

public class Runner {
    public static void main(String[] args) {
        CaptureCases cases = new CaptureCases();
        System.out.println(cases.add(3).applyAsInt(4) + ":" + cases.bound(2).getAsInt());
    }
}
