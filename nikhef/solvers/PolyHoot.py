from typing import Any, Dict, Literal, List
from copy import deepcopy
from optimizers import BaseOptimizer, pMap

class Node(object):
    def __init__(self, bounds, parent: Any = None):
        self.sum_reward = 0
        self.visited = 0
        self.children = []
        self.parent = parent
        self.bounds = bounds

    def add_child(self, child: Any):
        self.children.append(child)

class PolyHoot(BaseOptimizer):
    def __init__(
        self,
        max_iterations: int = 100,
        objective: Literal["min", "max"] = "min"
    ):
        super().__init__(objective=objective, auto_eval={"autoEval": True, "nested": True, "weights": [1.0, 1.0, 1.0, -10.0]})
        self.max_iterations = max_iterations
        self.current_iteration = 0


    def init(self) -> pMap:
        """
        Initializes with a random point within bounds.
        """

        self.best_score = 0


        self.cfg = self._algorithm.get_config()
        self.bounds = self._algorithm.get_bounds()

        self.alfa = 5
        self.epsilon = 20
        self.eta = 0.5

        self.root = Node(bounds=self.bounds)  # Root node with no bounds
        self.nodes = [self.root]
        self.current_node = self.root

        self.param_num = 0

        for key, (typ, _) in self.cfg.items():
            if typ is not bool:
                self.param_num += 1

        self.nu = 4 * self.param_num
        self.ro = 1 / (4 * self.param_num)



        
        self.current_iteration += 1

        #pregenerate trees of bounds
        for key, (typ, _) in self.cfg.items():
            if typ is bool:
                current_leaves = [node for node in self.nodes if len(node.children) == 0]
                for node in current_leaves: #NOTE: I think this is an infite loop no since we keep adding nodes to the nodes list in the loop we keep adding children forever no?
                    #also changed logic since we were overriding the original bounds. (unless you changed it already then no, since current leaves is a copy of the current nodes)
                    bounds_false = deepcopy(node.bounds)
                    bounds_false[key] = False #TODO should this be an int or a true bool to ask group.. (changing  it to bool)
                    node1 = Node(bounds=bounds_false, parent=node)
                    
                    bounds_true = deepcopy(node.bounds)
                    bounds_true[key] = True #TODO should this be an int or a true bool to ask group..
                    node2 = Node(bounds=bounds_true, parent=node)
                    
                    #I think we also want to add these as children to the parent node yeah? or in this case the current node (yes indeed)
                    node.children.append(node1)
                    node.children.append(node2)
                    
                    #leave unchanged.. (I don't think we ever use the node list again, but just in case)
                    self.nodes.append(node1)
                    self.nodes.append(node2)


        self.current_node = self.root
        depth = 0

        while len(self.current_node.children):
            self.current_node.visited += 1
            self.current_node = node.children[0]
            depth += 1


        self.current_node.visited += 1

        new_bounds = self.returnBounds(self.current_node.bounds)

        self.current_node.add_child(Node(bounds=new_bounds[0], parent=self.current_node))
        self.current_node.add_child(Node(bounds=new_bounds[1], parent=self.current_node))


        pmap = self.returnPmap(self.current_node.bounds)

        # print(f"Bounds: {self.current_node.bounds}\n pmap: {pmap}\n")

        return pmap
    


    def next(self) -> pMap:
        """
        Evaluates the current configuration and returns a new one.
        """
        self.current_iteration += 1


        # Evaluate the current configuration (from previous init/next call)
        #score = self.objective_func([1.0, 1.0, 1.0, -10.0])
        #so we already increased the count of the nodes but we still want to backprop the scors which should happen after a run so at the start of 
        # next we get the last score in history and trace back up the stack accordingly or just use the score above but faster to get the score from history??? TODO ask team
        #TODO:backprop here FIXED?!?!
        if hasattr(self, 'score_history') and len(self.score_history) > 0:
            score = self.score_history[-1]
            # print(score)
            current = self.current_node
            while current is not None:
                current.sum_reward += score
                current = current.parent
        
        
        # print(f"score: {self.current_node.sum_reward}\n")

        if self.current_node.sum_reward < self.best_score:
            self.best_score = self.current_node.sum_reward
        
        node = self.root
        depth = 0



        while len(node.children):
            node.visited += 1
            if node.children[0].visited == 0:
                node = node.children[0]
            elif node.children[1].visited == 0:
                node = node.children[1]
            else:
                node1_score = (-node.children[0].sum_reward / node.children[0].visited) + (self.current_iteration ** (self.alfa/self.epsilon)) * (node.children[0].visited ** (self.eta - 1)) + (self.nu * (self.ro ** depth))
                node2_score = (-node.children[1].sum_reward / node.children[1].visited) + (self.current_iteration ** (self.alfa/self.epsilon)) * (node.children[1].visited ** (self.eta - 1)) + (self.nu * (self.ro ** depth))

                if node1_score > node2_score:
                    node = node.children[0]
                else:
                    node = node.children[1]

            depth += 1

        node.visited += 1

        new_bounds = self.returnBounds(node.bounds)

        node.add_child(Node(bounds=new_bounds[0], parent=node))
        node.add_child(Node(bounds=new_bounds[1], parent=node))


        pmap = self.returnPmap(node.bounds)
        #keep track of the leaf node we expanded and rolled out for the backprop we we do next again.
        self.current_node = node

        # print(f"Bounds: {self.current_node.bounds}\n pmap: {pmap}\n")
            
        return pmap
    

    def is_finished(self) -> bool:
        """Determines if optimization is complete."""
        #TODO: possibly add a check for reaching target score
        #TODO check with team but I think here we also want to perform backprop since if were finished we wont performn the last next we need to backprop so we do it here instead (maybe, depends when this function is called by the pipeline(TODO: check it), either way, if we don't it would just mean we skip the last iteration, which should not be that big of a problem)
        finished = self.current_iteration >= self.max_iterations
        if finished:
            if hasattr(self, 'score_history') and len(self.score_history) > 0:
                score = self.score_history[-1]
                current = self.current_node
                while current is not None:
                    current.sum_reward += score
                    current = current.parent


        print(f"score: {self.best_score}\n")
                
        return finished
    

    #return pmap based on bounds (split in middle) (rollout phase)
    def returnPmap (self, bounds: Dict[str, Any]) -> Dict[str, Any]:
        new_pmap = {}
        
        for key, (typ, _) in self.cfg.items():
            if typ is float:
                new_pmap[key] = (bounds[key][0] + bounds[key][1]) / 2
            elif typ is int:
                new_pmap[key] = (int)((bounds[key][0] + bounds[key][1]) / 2)
            elif typ is bool:
                new_pmap[key] = bounds[key]
            
        return new_pmap


    def returnBounds(self, bounds: Dict[str, Any]) -> List [Dict[str, Any]]: #TODO we currently split each of the bounds in half but we only want to split the axis with the 
                                                                                            # currently largest diameter (yes, should be done)

                                                                                            # also its late my brain is cooked but should we skip over the keys that 
                                                                                            # are boolean as we currently try to split these? (I added that now, just in case)

                                                                                            # we also never noremalize anywhere. (will do that after lunch, will be done impleicetly (how do you write that?) when calculating biggest bound)

        map = self.returnPmap(bounds)
        new_bounds = [deepcopy(bounds), deepcopy(bounds)]

        
        # find biggest bound
        max_key = None
        max_relative_diff = -1

        for key, (typ, _) in self.cfg.items():
            if typ is not bool:
                original_range = self.bounds[key][1] - self.bounds[key][0]
                current_range = bounds[key][1] - bounds[key][0]
                relative_diff = current_range / original_range if original_range != 0 else 0

                if relative_diff > max_relative_diff:
                    max_relative_diff = relative_diff
                    max_key = key

        # Split only the key with the biggest relative size
        if max_key is not None:
            low, high = new_bounds[0][max_key]
            new_bounds[0][max_key] = (low, map[max_key])

            low, high = new_bounds[1][max_key]
            new_bounds[1][max_key] = (map[max_key], high)

            
        return new_bounds
            