import time
import papr

if __name__ == '__main__':
    graph = papr.GraphBuilder()
    out1 = graph.output()
    out2 = graph.output()
    rt = papr.Runtime(graph.build())
    rt.run()
    time.sleep(2)