package p;

public class Runner {
    public static void main(String[] args) {
        Base result = Subject.make();
        System.out.println(Subject.value);
        result.run();
        System.out.println(Subject.value);
    }
}
