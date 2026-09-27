package em23;

public class Runner {
    public static void main(String[] args) {
        Updates value = new Updates();
        value.increment();
        value.decrement();
        value.append("A");
        value.append("B");
        System.out.println(value.instanceField + ":" + Updates.staticField + ":" + Updates.result);
        System.out.println(value.plusTwo(3) + ":" + value.next(3));
    }
}
