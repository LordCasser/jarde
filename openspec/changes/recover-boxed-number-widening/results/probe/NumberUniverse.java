import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/// Task 1.1's reflective check of the direct-edge universe: which release-8 classes really declare
/// `extends java.lang.Number`, and which ones reach `Number` only through another class. The class
/// names come from the same `rt.jar` the tables' rows are transcribed from (`jar tf` on stdin), and
/// every name is loaded with `initialize = false` so no static initializer runs.
public class NumberUniverse {
    public static void main(String[] args) throws Exception {
        List<String> names = new ArrayList<String>();
        BufferedReader reader = new BufferedReader(new InputStreamReader(System.in));
        String line;
        while ((line = reader.readLine()) != null) {
            if (!line.isEmpty()) {
                names.add(line);
            }
        }
        List<String> javaLangDirect = new ArrayList<String>();
        List<String> javaLangIndirect = new ArrayList<String>();
        List<String> elsewhereDirect = new ArrayList<String>();
        List<String> elsewhereIndirect = new ArrayList<String>();
        for (String name : names) {
            Class<?> type;
            try {
                type = Class.forName(name, false, ClassLoader.getSystemClassLoader());
            } catch (Throwable ignored) {
                continue;
            }
            if (type == Number.class || !Number.class.isAssignableFrom(type)) {
                continue;
            }
            boolean javaLang = name.startsWith("java.lang.");
            if (type.getSuperclass() == Number.class) {
                (javaLang ? javaLangDirect : elsewhereDirect).add(name);
            } else {
                (javaLang ? javaLangIndirect : elsewhereIndirect).add(name);
            }
        }
        Collections.sort(javaLangDirect);
        Collections.sort(javaLangIndirect);
        Collections.sort(elsewhereDirect);
        Collections.sort(elsewhereIndirect);
        System.out.println("java.lang direct subclasses of Number (" + javaLangDirect.size() + "):");
        for (String name : javaLangDirect) {
            System.out.println("  " + name + " extends " + Class.forName(name, false, ClassLoader.getSystemClassLoader()).getSuperclass().getName());
        }
        System.out.println("java.lang indirect Number subclasses (" + javaLangIndirect.size() + "): " + javaLangIndirect);
        System.out.println("non-java.lang direct Number subclasses (" + elsewhereDirect.size() + "): " + elsewhereDirect);
        System.out.println("non-java.lang indirect Number subclasses (" + elsewhereIndirect.size() + "): " + elsewhereIndirect);
        System.out.println("Number's own superclass: " + Number.class.getSuperclass().getName());
        System.out.println("Number's own interfaces: " + java.util.Arrays.toString(Number.class.getInterfaces()));
    }
}
