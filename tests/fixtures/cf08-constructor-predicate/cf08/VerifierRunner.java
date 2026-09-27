package cf08;

import java.io.File;
import java.lang.reflect.Method;

public final class VerifierRunner {
    public static void main(String[] args) throws Exception {
        File[] files = {new File("f")};
        String[] names = {"constructorAliasStore", "constructorExtraConsumer",
                "constructorEffectArgument", "predicateExtraCall", "predicateOtherReceiver",
                "predicateExtraEffect", "extraInnerExit", "innerJoinEffect", "outerNullObject"};
        for (String name : names) {
            Method method = NotIndexedLoopNegatives.class.getDeclaredMethod(name, File[].class);
            File[] input = name.equals("outerNullObject") ? null : files;
            File result = (File) method.invoke(null, new Object[] {input});
            System.out.println(name + ":" + (result == null ? "null" : result.getName()));
        }
    }
}
