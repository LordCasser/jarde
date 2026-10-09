import java.io.File;
import java.lang.reflect.Method;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Paths;

/**
 * Load and resolve a variant Main without initializing it or invoking its methods.
 * The variant directory contains Main.class and all unchanged companion classes.
 */
public final class VerifyOnly {
    private static final class VariantLoader extends URLClassLoader {
        VariantLoader(File variantDir) throws Exception {
            super(new URL[] { variantDir.toURI().toURL() }, null);
        }

        Class<?> loadResolveAndInspect() throws ClassNotFoundException {
            Class<?> main = Class.forName("Main", false, this);
            resolveClass(main);
            Method[] methods = main.getDeclaredMethods();
            if (methods.length == 0) {
                throw new AssertionError("expected Main methods");
            }
            System.out.println("VERIFY_OK class=" + main.getName()
                    + " methods=" + methods.length);
            return main;
        }
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("<variant-dir> <valid|invalid>");
        }
        final boolean invalidExpected;
        if ("invalid".equals(args[1])) {
            invalidExpected = true;
        } else if ("valid".equals(args[1])) {
            invalidExpected = false;
        } else {
            throw new IllegalArgumentException("mode must be valid or invalid");
        }

        try (VariantLoader loader = new VariantLoader(Paths.get(args[0]).toFile())) {
            try {
                loader.loadResolveAndInspect();
                if (invalidExpected) {
                    throw new AssertionError("invalid Code unexpectedly verified");
                }
            } catch (VerifyError error) {
                System.err.println("VERIFY_ERROR " + error.getClass().getName()
                        + ": " + error.getMessage());
                error.printStackTrace(System.err);
                if (!invalidExpected) {
                    throw error;
                }
            }
        }
    }
}
