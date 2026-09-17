export namespace host {
	
	export class Group {
	    id: string;
	    name: string;
	    sort_order: number;
	    created_at: string;
	    updated_at: string;
	    deleted_at?: string;
	
	    static createFrom(source: any = {}) {
	        return new Group(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.sort_order = source["sort_order"];
	        this.created_at = source["created_at"];
	        this.updated_at = source["updated_at"];
	        this.deleted_at = source["deleted_at"];
	    }
	}
	export class Host {
	    id: string;
	    group_id?: string;
	    name: string;
	    hostname: string;
	    port: number;
	    username: string;
	    auth_type: string;
	    created_at: string;
	    updated_at: string;
	    deleted_at?: string;
	
	    static createFrom(source: any = {}) {
	        return new Host(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.group_id = source["group_id"];
	        this.name = source["name"];
	        this.hostname = source["hostname"];
	        this.port = source["port"];
	        this.username = source["username"];
	        this.auth_type = source["auth_type"];
	        this.created_at = source["created_at"];
	        this.updated_at = source["updated_at"];
	        this.deleted_at = source["deleted_at"];
	    }
	}
	export class HostInput {
	    group_id?: string;
	    name: string;
	    hostname: string;
	    port: number;
	    username: string;
	    auth_type: string;
	    password: string;
	    private_key: string;
	
	    static createFrom(source: any = {}) {
	        return new HostInput(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.group_id = source["group_id"];
	        this.name = source["name"];
	        this.hostname = source["hostname"];
	        this.port = source["port"];
	        this.username = source["username"];
	        this.auth_type = source["auth_type"];
	        this.password = source["password"];
	        this.private_key = source["private_key"];
	    }
	}

}

export namespace sync {
	
	export class MergeResult {
	    added: number;
	    updated: number;
	    deleted: number;
	    skipped: string[];
	    conflicts: string[];
	
	    static createFrom(source: any = {}) {
	        return new MergeResult(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.added = source["added"];
	        this.updated = source["updated"];
	        this.deleted = source["deleted"];
	        this.skipped = source["skipped"];
	        this.conflicts = source["conflicts"];
	    }
	}

}

