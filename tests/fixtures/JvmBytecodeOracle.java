import java.io.InputStream;
import java.lang.classfile.ClassFile;
import java.lang.classfile.ClassModel;
import java.lang.classfile.CodeModel;
import java.lang.classfile.Instruction;
import java.lang.classfile.MethodModel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.Set;

class OracleFixture {
    static int dense(int value) {
        switch (value) {
            case 0: return 10;
            case 1: return 11;
            case 2: return 12;
            case 3: return 13;
            default: return -1;
        }
    }

    static int sparse(int value) {
        switch (value) {
            case -100: return 1;
            case 7: return 2;
            case 1000: return 3;
            default: return -1;
        }
    }

    static Object refs() {
        return new Object();
    }
}

public class JvmBytecodeOracle {
    private static final Set<String> TARGETS = Set.of("dense", "sparse", "refs");

    public static void main(String[] args) throws Exception {
        if (args.length != 1) {
            throw new IllegalArgumentException("expected output class path");
        }
        byte[] bytes;
        try (InputStream input = JvmBytecodeOracle.class.getClassLoader()
                .getResourceAsStream("OracleFixture.class")) {
            if (input == null) {
                throw new IllegalStateException("OracleFixture.class resource unavailable");
            }
            bytes = input.readAllBytes();
        }
        bytes[4] = 0;
        bytes[5] = 0;
        bytes[6] = 0;
        bytes[7] = 52;
        Files.write(Path.of(args[0]), bytes);

        String sha256 = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        System.out.println("META\truntime\t" + Runtime.version());
        System.out.println("META\tsha256\t" + sha256);
        System.out.println("META\tversion\t52\t0");
        System.out.println("META\tscope\tinstruction_boundary_only_not_verification");

        ClassModel model = ClassFile.of().parse(bytes);
        for (MethodModel method : model.methods()) {
            String name = method.methodName().stringValue();
            if (!TARGETS.contains(name)) {
                continue;
            }
            String descriptor = method.methodType().stringValue();
            CodeModel code = method.code().orElseThrow();
            int bci = 0;
            for (var element : code) {
                if (element instanceof Instruction instruction) {
                    int width = instruction.sizeInBytes();
                    System.out.println("FACT\t" + name + "\t" + descriptor + "\t" + bci
                            + "\t" + instruction.opcode().bytecode() + "\t" + width);
                    bci += width;
                }
            }
            System.out.println("END\t" + name + "\t" + descriptor + "\t" + bci);
        }
    }
}
