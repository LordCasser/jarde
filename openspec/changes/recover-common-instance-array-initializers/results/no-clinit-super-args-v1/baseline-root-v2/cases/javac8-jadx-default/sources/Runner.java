package defpackage;

import java.util.Arrays;

public class Runner {
    public static void main(String[] args) {
        CommonNoClinitArrayInit.trace = 0;
        CommonNoClinitArrayInit noArgs = new CommonNoClinitArrayInit();
        System.out.println("noargs-super=" + noArgs.received);
        System.out.println("noargs-arrays=" + Arrays.toString(noArgs.first) + "/" + Arrays.toString(noArgs.second));
        System.out.println("noargs-trace=" + CommonNoClinitArrayInit.trace);

        CommonNoClinitArrayInit.trace = 0;
        CommonNoClinitArrayInit withArg = new CommonNoClinitArrayInit(40);
        System.out.println("arg-super=" + withArg.received);
        System.out.println("arg-arrays=" + Arrays.toString(withArg.first) + "/" + Arrays.toString(withArg.second));
        System.out.println("arg-trace=" + CommonNoClinitArrayInit.trace);

        CommonNoClinitArrayInit another = new CommonNoClinitArrayInit(40);
        System.out.println("fresh=" + (withArg.first != another.first && withArg.second != another.second));
    }
}
