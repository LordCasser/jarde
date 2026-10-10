import java.util.Arrays;

public class Runner {
    public static void main(String[] args) {
        ByteArrayReturn source = new ByteArrayReturn();
        byte[] first = source.test();
        byte[] second = source.test();
        System.out.println(Arrays.toString(first));
        System.out.println(first == second);
        first[0] = 9;
        System.out.println(Arrays.toString(source.test()));
    }
}
