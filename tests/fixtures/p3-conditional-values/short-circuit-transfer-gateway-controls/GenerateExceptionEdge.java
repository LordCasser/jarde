import java.nio.file.*;
import jdk.internal.org.objectweb.asm.*;
import jdk.internal.org.objectweb.asm.tree.*;
public class GenerateExceptionEdge {
 public static void main(String[] args) throws Exception {
  ClassNode node=new ClassNode(); new ClassReader(Files.readAllBytes(Path.of(args[0]))).accept(node,0);
  for (MethodNode m:node.methods) if(m.name.equals("assign")) {
   LabelNode start=new LabelNode(); LabelNode end=new LabelNode(); LabelNode handler=new LabelNode();
   AbstractInsnNode gateway=m.instructions.get(4); AbstractInsnNode next=m.instructions.get(5);
   m.instructions.insertBefore(gateway,start);
   m.instructions.insertBefore(next,end);
   m.instructions.add(handler); m.instructions.add(new InsnNode(Opcodes.POP)); m.instructions.add(new InsnNode(Opcodes.RETURN));
   m.tryCatchBlocks.add(new TryCatchBlockNode(start,end,handler,"java/lang/Throwable"));
  }
  ClassWriter cw=new ClassWriter(ClassWriter.COMPUTE_FRAMES|ClassWriter.COMPUTE_MAXS);
  node.accept(cw); Files.write(Path.of(args[1]),cw.toByteArray());
 }
}
