public class ArrayReferenceConversionsRunner {
    private static String error(Throwable error) {
        return error.getClass().getName();
    }

    private static void line(String name, String value) {
        System.out.println(name + "=" + value);
    }

    private static void call(String name, Call call) {
        try {
            line(name, call.run());
        } catch (Throwable error) {
            line(name, "error:" + error(error));
        }
    }

    private interface Call {
        String run();
    }

    private static void runTypedTargets(final int[][] intMatrix, final String[] strings, final String[][] stringMatrix) {
        call("target:int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.typedIntMatrix(intMatrix);
            }
        });
        call("target:String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.typedStringArray(strings);
            }
        });
        call("target:String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.typedStringMatrix(stringMatrix);
            }
        });
        call("target:Object-widened-int[][]", new Call() {
            public String run() {
                Object value = intMatrix;
                return ArrayReferenceConversions.widenedObject(value);
            }
        });
        call("target:Object[]-widened-int[][]", new Call() {
            public String run() {
                Object[] value = intMatrix;
                return ArrayReferenceConversions.widenedObjectArray(value);
            }
        });
        call("target:Object-widened-String[]", new Call() {
            public String run() {
                Object value = strings;
                return ArrayReferenceConversions.widenedObject(value);
            }
        });
        call("target:Object[]-widened-String[]", new Call() {
            public String run() {
                Object[] value = strings;
                return ArrayReferenceConversions.widenedObjectArray(value);
            }
        });
        call("target:Object-widened-String[][]", new Call() {
            public String run() {
                Object value = stringMatrix;
                return ArrayReferenceConversions.widenedObject(value);
            }
        });
        call("target:Object[]-widened-String[][]", new Call() {
            public String run() {
                Object[] value = stringMatrix;
                return ArrayReferenceConversions.widenedObjectArray(value);
            }
        });
    }

    private static void runExact(final int[][] intMatrix, final String[] strings, final String[][] stringMatrix) {
        call("exact:int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactIntMatrix(intMatrix);
            }
        });
        call("exact:String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactStringArray(strings);
            }
        });
        call("exact:String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactStringMatrix(stringMatrix);
            }
        });
        call("exact:Object[]-int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactObjectArray(intMatrix);
            }
        });
        call("exact:Object[]-String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactObjectArray(strings);
            }
        });
        call("exact:Object[]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactObjectArray(stringMatrix);
            }
        });
        call("exact:Object[][]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactObjectMatrix(stringMatrix);
            }
        });
        call("describe:Object[]-int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.describeObjectArray(intMatrix);
            }
        });
        call("describe:Object[]-String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.describeObjectArray(strings);
            }
        });
        call("describe:Object[][]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.describeObjectMatrix(stringMatrix);
            }
        });
    }

    private static void runWrites(final int[][] intMatrix, final String[] strings, final String[][] stringMatrix) {
        call("write:Object[]-int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectArray(intMatrix);
            }
        });
        call("write:Object[]-String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectArray(strings);
            }
        });
        call("write:Object[][]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectMatrix(stringMatrix);
            }
        });
        call("write:Object[]-empty-int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectArray(new int[0][]);
            }
        });
        call("write:Object[]-empty-String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectArray(new String[0]);
            }
        });
        call("write:Object[][]-empty-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.writeObjectMatrix(new String[0][]);
            }
        });
    }

    private static void runSizes(final int[][] intMatrix, final String[] strings, final String[][] stringMatrix) {
        line("size:int[][]", ArrayReferenceConversions.exactIntMatrix(intMatrix));
        line("size:String[]", ArrayReferenceConversions.exactStringArray(strings));
        line("size:String[][]", ArrayReferenceConversions.exactStringMatrix(stringMatrix));
        line("size:Object[]-int[][]", ArrayReferenceConversions.describeObjectArray(intMatrix));
        line("size:Object[]-String[]", ArrayReferenceConversions.describeObjectArray(strings));
        line("size:Object[][]-String[][]", ArrayReferenceConversions.describeObjectMatrix(stringMatrix));
    }

    private static void runNulls() {
        final int[][] intNull = null;
        final String[] stringNull = null;
        final String[][] stringMatrixNull = null;
        call("null:exact-int[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactIntMatrix(intNull);
            }
        });
        call("null:exact-String[]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactStringArray(stringNull);
            }
        });
        call("null:exact-String[][]", new Call() {
            public String run() {
                return ArrayReferenceConversions.exactStringMatrix(stringMatrixNull);
            }
        });
        call("null:Object-widened", new Call() {
            public String run() {
                Object value = intNull;
                return ArrayReferenceConversions.widenedObject(value);
            }
        });
        call("null:Object[]-widened", new Call() {
            public String run() {
                Object[] value = stringMatrixNull;
                return ArrayReferenceConversions.widenedObjectArray(value);
            }
        });
        call("null:Object[]-dereference", new Call() {
            public String run() {
                return ArrayReferenceConversions.describeObjectArray(stringNull);
            }
        });
    }

    public static void main(String[] args) {
        int[][] intMatrix = new int[2][];
        intMatrix[0] = new int[0];
        intMatrix[1] = new int[3];
        String[] strings = new String[2];
        String[][] stringMatrix = new String[2][];
        stringMatrix[0] = new String[0];
        stringMatrix[1] = new String[4];
        runTypedTargets(intMatrix, strings, stringMatrix);
        runExact(intMatrix, strings, stringMatrix);
        runWrites(intMatrix, strings, stringMatrix);
        runSizes(intMatrix, strings, stringMatrix);
        runNulls();
    }
}
