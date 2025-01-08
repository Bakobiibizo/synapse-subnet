import sys
import json
import os
from communex import CommuneClient

def get_client():
    url = os.environ.get('COMMUNE_RPC_URL')
    if not url:
        raise ValueError("COMMUNE_RPC_URL environment variable not set")
    return CommuneClient(url=url)

def handle_register_module(args):
    module_data = json.loads(args[0])
    netuid = int(args[1])
    
    client = get_client()
    # TODO: Need to implement module registration using communex
    # This is a placeholder until we determine the exact API
    return json.dumps({"success": True})

def handle_get_module(args):
    name = args[0]
    netuid = int(args[1])
    
    client = get_client()
    # Get module info using communex client
    # TODO: Need to determine exact API for getting module info
    module = {
        "name": name,
        "address": "",  # TODO: Get from network
        "stake": 0,     # TODO: Get from network
        "metadata": None
    }
    return json.dumps(module)

def handle_list_modules(args):
    netuid = int(args[0])
    
    client = get_client()
    # TODO: Need to determine exact API for listing modules
    modules = []
    return json.dumps(modules)

def handle_stake(args):
    module_name = args[0]
    amount = int(args[1])
    netuid = int(args[2])
    
    client = get_client()
    # TODO: Need to determine exact API for staking
    return json.dumps({"success": True})

def handle_unstake(args):
    module_name = args[0]
    amount = int(args[1])
    netuid = int(args[2])
    
    client = get_client()
    # TODO: Need to determine exact API for unstaking
    return json.dumps({"success": True})

def handle_get_stake(args):
    module_name = args[0]
    netuid = int(args[1])
    
    client = get_client()
    # TODO: Need to determine exact API for getting stake
    return json.dumps(0)

def handle_get_min_stake(args):
    netuid = int(args[0])
    
    client = get_client()
    min_stake = client.get_min_stake(netuid)
    return json.dumps(min_stake)

def handle_get_max_allowed_modules(args):
    client = get_client()
    max_modules = client.get_max_allowed_modules()
    return json.dumps(max_modules)

def main():
    if len(sys.argv) < 2:
        print("Error: No command specified", file=sys.stderr)
        sys.exit(1)
        
    command = sys.argv[1]
    args = sys.argv[2:]
    
    try:
        if command == "register_module":
            result = handle_register_module(args)
        elif command == "get_module":
            result = handle_get_module(args)
        elif command == "list_modules":
            result = handle_list_modules(args)
        elif command == "stake":
            result = handle_stake(args)
        elif command == "unstake":
            result = handle_unstake(args)
        elif command == "get_stake":
            result = handle_get_stake(args)
        elif command == "get_min_stake":
            result = handle_get_min_stake(args)
        elif command == "get_max_allowed_modules":
            result = handle_get_max_allowed_modules(args)
        else:
            print(f"Error: Unknown command {command}", file=sys.stderr)
            sys.exit(1)
            
        print(result)
        sys.exit(0)
    except Exception as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
