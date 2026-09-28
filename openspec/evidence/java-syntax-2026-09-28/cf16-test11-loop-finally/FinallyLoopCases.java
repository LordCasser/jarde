import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.AbstractList;
import java.util.Iterator;
import java.util.List;

public class FinallyLoopCases {
    private static List<Object> values(final String mode) {
        return new AbstractList<Object>() {
            @Override public Object get(int index) { return index == 0 ? "1" : "2"; }
            @Override public int size() { return 2; }
            @Override public Iterator<Object> iterator() {
                if (mode.equals("iterator")) throw new IllegalStateException("iterator");
                return new Iterator<Object>() {
                    private int index;
                    @Override public boolean hasNext() {
                        if (mode.equals("hasNext")) throw new IllegalStateException("hasNext");
                        return index < 2;
                    }
                    @Override public Object next() {
                        if (mode.equals("next")) throw new IllegalStateException("next");
                        return get(index++);
                    }
                };
            }
        };
    }

    public static void main(String[] args) throws Exception {
        String className = args[0];
        String mode = args[1];
        boolean bodyFails = Boolean.parseBoolean(args[2]);
        Class<?> cls = Class.forName(className);
        Object instance = cls.getDeclaredConstructor().newInstance();
        cls.getField("fail").setBoolean(null, bodyFails);
        cls.getField("failCleanup").setBoolean(null, mode.equals("call2"));
        Method test = cls.getMethod("test", List.class);
        String result;
        try {
            test.invoke(instance, values(mode));
            result = "ok";
        } catch (InvocationTargetException failure) {
            Throwable thrown = failure.getCause();
            result = thrown.getClass().getSimpleName() + ":" + thrown.getMessage();
        }
        int count = ((Integer) cls.getMethod("count").invoke(instance)).intValue();
        System.out.println(mode + ":" + bodyFails + ":" + count + ":" + result);
    }
}
