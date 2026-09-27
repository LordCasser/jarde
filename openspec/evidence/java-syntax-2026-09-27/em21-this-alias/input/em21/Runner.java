package em21;

public class Runner {
    public static void main(String[] args) {
        ThisUse a = new ThisUse();
        a.inline();
        System.out.println(a.field + ":" + a.touches());
        a.checked();
        System.out.println(a.field + ":" + a.touches());
        a.field = 7;
        ThisUse same = a.choose();
        System.out.println((same == a) + ":" + same.touches());
        a.field = 8;
        ThisUse other = a.choose();
        System.out.println((other != a) + ":" + other.touches());
    }
}
