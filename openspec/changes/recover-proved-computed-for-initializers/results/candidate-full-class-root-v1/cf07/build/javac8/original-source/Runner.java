package cf07;

public class Runner {
    public static void main(String[] args) {
        int[] values = {1, 2, 3, 2};
        System.out.println(LoopCases.andWhile(false));
        System.out.println(LoopCases.andWhile(true));
        System.out.println(LoopCases.counted(1, 4));
        System.out.println(LoopCases.counted(6, 9));
        System.out.println(LoopCases.counted(5, 5));
        System.out.println(LoopCases.lastIndexOf(values, 2, 0, 4));
        System.out.println(LoopCases.lastIndexOf(values, 4, 0, 4));
        System.out.println(LoopCases.lastIndexOf(values, 2, 2, 4));
    }
}
