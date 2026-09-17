export namespace audit {
	
	export class AuditLog {
	    id: string;
	    host_id: string;
	    command: string;
	    output: string;
	    exit_code: number;
	    // Go type: time
	    created_at: any;
	
	    static createFrom(source: any = {}) {
	        return new AuditLog(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.host_id = source["host_id"];
	        this.command = source["command"];
	        this.output = source["output"];
	        this.exit_code = source["exit_code"];
	        this.created_at = this.convertValues(source["created_at"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}

}

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

export namespace snippet {
	
	export class Snippet {
	    id: string;
	    name: string;
	    body: string;
	    autoEnter: number;
	    scopeType: string;
	    scopeId: string;
	    // Go type: time
	    createdAt: any;
	    // Go type: time
	    updatedAt: any;
	
	    static createFrom(source: any = {}) {
	        return new Snippet(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.body = source["body"];
	        this.autoEnter = source["autoEnter"];
	        this.scopeType = source["scopeType"];
	        this.scopeId = source["scopeId"];
	        this.createdAt = this.convertValues(source["createdAt"], null);
	        this.updatedAt = this.convertValues(source["updatedAt"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class SnippetInput {
	    name: string;
	    body: string;
	    autoEnter: number;
	    scopeType: string;
	    scopeId: string;
	
	    static createFrom(source: any = {}) {
	        return new SnippetInput(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.body = source["body"];
	        this.autoEnter = source["autoEnter"];
	        this.scopeType = source["scopeType"];
	        this.scopeId = source["scopeId"];
	    }
	}

}

export namespace sshkey {
	
	export class SSHKey {
	    id: string;
	    name: string;
	    private_key?: string;
	    public_key: string;
	    fingerprint: string;
	    // Go type: time
	    created_at: any;
	    // Go type: time
	    updated_at: any;
	
	    static createFrom(source: any = {}) {
	        return new SSHKey(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.private_key = source["private_key"];
	        this.public_key = source["public_key"];
	        this.fingerprint = source["fingerprint"];
	        this.created_at = this.convertValues(source["created_at"], null);
	        this.updated_at = this.convertValues(source["updated_at"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
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

export namespace team {
	
	export class Team {
	    id: string;
	    name: string;
	    // Go type: time
	    createdAt: any;
	    // Go type: time
	    updatedAt: any;
	
	    static createFrom(source: any = {}) {
	        return new Team(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.createdAt = this.convertValues(source["createdAt"], null);
	        this.updatedAt = this.convertValues(source["updatedAt"], null);
	    }
	
		convertValues(a: any, classs: any, asMap: boolean = false): any {
		    if (!a) {
		        return a;
		    }
		    if (a.slice && a.map) {
		        return (a as any[]).map(elem => this.convertValues(elem, classs));
		    } else if ("object" === typeof a) {
		        if (asMap) {
		            for (const key of Object.keys(a)) {
		                a[key] = new classs(a[key]);
		            }
		            return a;
		        }
		        return new classs(a);
		    }
		    return a;
		}
	}
	export class TeamInput {
	    name: string;
	
	    static createFrom(source: any = {}) {
	        return new TeamInput(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	    }
	}

}

