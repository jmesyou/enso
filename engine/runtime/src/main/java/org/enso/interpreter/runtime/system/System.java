package org.enso.interpreter.runtime.system;

import com.oracle.truffle.api.CompilerDirectives;
import com.oracle.truffle.api.dsl.Cached;
import com.oracle.truffle.api.io.TruffleProcessBuilder;
import org.apache.commons.lang3.SystemUtils;
import org.enso.interpreter.dsl.Builtin;
import org.enso.interpreter.node.expression.builtin.text.util.ExpectStringNode;
import org.enso.interpreter.runtime.Context;
import org.enso.interpreter.runtime.callable.atom.Atom;
import org.enso.interpreter.runtime.data.Array;
import org.enso.interpreter.runtime.data.text.Text;
import org.enso.interpreter.runtime.error.PanicException;

import java.io.*;

public class System {

  private static final Text LINUX = Text.create("linux");
  private static final Text MACOS = Text.create("macos");
  private static final Text WINDOWS = Text.create("windows");
  private static final Text UNKNOWN = Text.create("unknown");

  @Builtin.Method(description = "Get the type of operating system.")
  @CompilerDirectives.TruffleBoundary
  public static Text os() {
    if (SystemUtils.IS_OS_LINUX) return LINUX;
    if (SystemUtils.IS_OS_MAC_OSX) return MACOS;
    if (SystemUtils.IS_OS_WINDOWS) return WINDOWS;
    return UNKNOWN;
  }

  @Builtin.Method(description = "Gets the nanosecond resolution system time.")
  @CompilerDirectives.TruffleBoundary
  public static void nanoTime() {
    java.lang.System.nanoTime();
  }

  @Builtin.Method(description = "Exits the process, returning the provided code.")
  @CompilerDirectives.TruffleBoundary
  public static void exit(long code) {
    java.lang.System.exit((int) code);
  }

  @Builtin.Specialize
  @Builtin.Method(description = "Create a system process, returning the exit code.")
  @Builtin.WrapException(from = IOException.class, to = PanicException.class, propagate = true)
  @Builtin.WrapException(
      from = InterruptedException.class,
      to = PanicException.class,
      propagate = true)
  @CompilerDirectives.TruffleBoundary
  public static Atom createProcess(
      Context ctx,
      Object command,
      Array arguments,
      Object input,
      boolean redirectIn,
      boolean redirectOut,
      boolean redirectErr,
      @Cached ExpectStringNode expectStringNode)
      throws IOException, InterruptedException {
    String[] cmd = new String[arguments.getItems().length + 1];
    cmd[0] = expectStringNode.execute(command);
    for (int i = 1; i <= arguments.getItems().length; i++) {
      cmd[i] = expectStringNode.execute(arguments.getItems()[i - 1]);
    }
    TruffleProcessBuilder pb = ctx.getEnvironment().newProcessBuilder(cmd);

    Process p = pb.start();
    ByteArrayInputStream in = new ByteArrayInputStream(expectStringNode.execute(input).getBytes());
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    ByteArrayOutputStream err = new ByteArrayOutputStream();

    boolean startedWritingtoOut = false;
    try (OutputStream processIn = p.getOutputStream()) {
      InputStream stdin;
      if (redirectIn) {
        stdin = ctx.getIn();
      } else {
        stdin = in;
      }
      int nread;
      startedWritingtoOut = true;
      byte[] buf = new byte[8096];
      while (stdin.available() > 0 && (nread = stdin.read(buf)) != -1) {
        processIn.write(buf, 0, nread);
      }
    } catch (IOException e) {
      // Getting the output stream of a finished process results in an IOException.
      // We can ignore it at this point.
      // Unless this exception is from writing to buffer/reading from stdin.
      if (startedWritingtoOut) throw e;
    }

    p.waitFor();

    try (InputStream processOut = p.getInputStream()) {
      OutputStream stdout;
      if (redirectOut) {
        stdout = ctx.getOut();
      } else {
        stdout = out;
      }
      int nread;
      byte[] buf = new byte[8096];
      while ((nread = processOut.read(buf)) != -1) {
        stdout.write(buf, 0, nread);
      }
    }

    try (InputStream processErr = p.getErrorStream()) {
      OutputStream stderr;
      if (redirectErr) {
        stderr = ctx.getErr();
      } else {
        stderr = err;
      }
      int nread;
      byte[] buf = new byte[8096];
      while ((nread = processErr.read(buf)) != -1) {
        stderr.write(buf, 0, nread);
      }
    }

    long exitCode = p.exitValue();
    Text returnOut = Text.create(out.toString());
    Text returnErr = Text.create(err.toString());

    return ctx.getBuiltins().system().makeSystemResult(exitCode, returnOut, returnErr);
  }
}