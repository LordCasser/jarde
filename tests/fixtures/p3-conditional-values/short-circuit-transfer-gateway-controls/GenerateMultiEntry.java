import java.nio.file.*;
import jdk.internal.org.objectweb.asm.*;
import jdk.internal.org.objectweb.asm.tree.*;
public class GenerateMultiEntry {
 public static void main(String[] args) throws Exception {
  byte[] bytes=Files.readAllBytes(Path.of(args[0]));
  ClassNode node=new ClassNode(); new ClassReader(bytes).accept(node,0);
  for (MethodNode m: node.methods) if (m.name.equals("assign")) {
   JumpInsnNode first=(JumpInsnNode)m.instructions.get(8);
   AbstractInsnNode gateway=m.instructions.get(4);
   LabelNode entry=new LabelNode();
   m.instructions.insertBefore(gateway, entry);
   first.label=entry;
  }
  ClassWriter cw=new ClassWriter(ClassWriter.COMPUTE_FRAMES|ClassWriter.COMPUTE_MAXS);
  node.accept(cw); Files.write(Path.of(args[1]), cw.toByteArray());
 }
}
