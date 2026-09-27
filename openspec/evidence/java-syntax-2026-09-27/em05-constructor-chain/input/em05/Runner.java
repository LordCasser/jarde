package em05;
public class Runner {
    public static void main(String[] args) {
        System.out.println(Factory.choose(true).value);
        System.out.println(Factory.choose(false).value);
        System.out.println(Factory.chain("c").value);
        System.out.println(new Child(5).value);
    }
}
