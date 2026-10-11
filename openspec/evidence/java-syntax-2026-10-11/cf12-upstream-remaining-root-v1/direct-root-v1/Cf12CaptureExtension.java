package cf12capture;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.lang.reflect.Field;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.extension.AfterTestExecutionCallback;
import org.junit.jupiter.api.extension.ExtensionContext;
import jadx.api.JadxDecompiler;
import jadx.api.JavaClass;
import jadx.tests.api.IntegrationTest;

/** Observation only; retains actual test inputs and generated code before Harness closes. */
public final class Cf12CaptureExtension implements AfterTestExecutionCallback {
    @Override
    public void afterTestExecution(ExtensionContext context) throws Exception {
        if (!(context.getRequiredTestInstance() instanceof IntegrationTest)) return;
        IntegrationTest test = (IntegrationTest) context.getRequiredTestInstance();
        String label = context.getRequiredTestClass().getSimpleName() + "." + context.getRequiredTestMethod().getName();
        Path out = Path.of(System.getProperty("cf12.capture.dir"), label);
        Files.createDirectory(out);
        List<String> paths = new ArrayList<>();
        for (File input : test.getArgs().getInputFiles()) {
            Path src = input.toPath();
            Path dst = out.resolve("input").resolve(input.getName());
            Files.createDirectories(dst.getParent());
            Files.copy(src, dst);
            paths.add(src.toAbsolutePath() + "\t" + dst.toAbsolutePath());
        }
        if (paths.isEmpty()) throw new AssertionError("Actual Harness input is empty: " + label);
        Files.write(out.resolve("actual-input-paths.tsv"), paths, StandardCharsets.UTF_8);
        Files.writeString(out.resolve("actual-jadx-args.txt"), test.getArgs().toString(), StandardCharsets.UTF_8);
        Field field = IntegrationTest.class.getDeclaredField("jadxDecompiler");
        field.setAccessible(true);
        JadxDecompiler decompiler = (JadxDecompiler) field.get(test);
        for (JavaClass cls : decompiler.getClasses()) {
            Path dst = out.resolve("jadx-source").resolve(cls.getFullName().replace('.', '/') + ".java");
            Files.createDirectories(dst.getParent());
            Files.writeString(dst, cls.getCode(), StandardCharsets.UTF_8);
        }
        Files.writeString(out.resolve("test-status.txt"), context.getExecutionException().isPresent() ? "FAILED\n" : "PASSED\n", StandardCharsets.UTF_8);
    }
}
