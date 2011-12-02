#!/usr/bin/python

import sys

# Graph represented as inverted adjacency list. Each vertex has a list of the
# vertices that point to it.
class Graph:
  def __init__(self, edges):
    self._graph = {}
    for (vertex2, vertex1) in edges:
      if vertex1 in self._graph:
        self._graph[vertex1].add(vertex2)
      else:
        self._graph[vertex1] = set(vertex2)
      if vertex2 not in self._graph:
        self._graph[vertex2] = set()

  def size(self):
    return len(self._graph)

  def no_incoming_edges_vertices(self):
    vertices = []
    for vertex, adjacencies in self._graph.iteritems():
      if len(adjacencies) == 0:
        vertices.append(vertex)
    return vertices

  def remove(self, vertex):
    if vertex in self._graph:
      del self._graph[vertex]
      for adjacencies in self._graph.values():
        if vertex in adjacencies:
          adjacencies.remove(vertex)

def partial_orders(words):
  for prev_word, word in zip(sys.argv[1:], sys.argv[2:]):
    for prev_char, char in zip(prev_word, word):
      if prev_char != char:
        yield (prev_char, char)
        break

if __name__ == "__main__":
  ordering_graph = Graph(partial_orders(sys.argv[1:]))
  while ordering_graph.size() > 0:
    vertices = ordering_graph.no_incoming_edges_vertices()
    if len(vertices) >= 1:
      print vertices
      for vertex in vertices:
        ordering_graph.remove(vertex)
    elif len(vertices) < 1:
      raise Exception("contradictory")
