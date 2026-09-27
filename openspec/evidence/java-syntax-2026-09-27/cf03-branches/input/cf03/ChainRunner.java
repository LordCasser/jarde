package cf03;

public class ChainRunner {
    public static void main(String[] args) {
        for (String value : new String[]{"a", "b", "3", "$", "z"}) {
            int result = ChainOnly.chain(value);
            System.out.println(result + ":" + ChainOnly.hits);
        }
    }
}
