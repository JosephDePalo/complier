local function is_bsd(conn)
	local re = regex.compile("NetBSD")
	local cmd_out = conn:run_cmd("uname -a")

	if re:is_match(cmd_out) then
		return true, "SSH configuration is secure."
	else
		return false, "Not NetBSD"
	end
end

local function has_ufw(conn)
	local cmd_out = conn:run_cmd('dpkg-query -s ufw &>/dev/null && echo "good"')
	if string.find(cmd_out, "good") then
		return true, "ufw installed"
	else
		return false, "ufw not installed"
	end
end

local function one_fw(conn)
	local cmd_out = conn:run_cmd([[
	   active_firewall=() firewalls=("ufw" "nftables" "iptables")
	   # Determine which firewall is in use
	   for firewall in "${firewalls[@]}"; do
	   case $firewall in
	   nftables)
	   cmd="nft" ;;
	   *)
	   cmd=$firewall ;;
	   esac
	   if command -v $cmd &> /dev/null && systemctl is-enabled --quiet\
	   $firewall && systemctl is-active --quiet $firewall; then
	   active_firewall+=("$firewall")
	   fi
	   done
	   # Display audit results
	   if [ ${#active_firewall[@]} -eq 1 ]; then
	   printf '%s\n' "" "Audit Results:" " ** PASS **" " - A single firewall
	   is in use follow the recommendation in ${active_firewall[0]} subsection ONLY"
	   elif [ ${#active_firewall[@]} -eq 0 ]; then
	   printf '%s\n' "" " Audit Results:" " ** FAIL **" "- No firewall in use
	   or unable to determine firewall status"
	   else
	   printf '%s\n' "" " Audit Results:" " ** FAIL **" " - Multiple firewalls
	   are in use: ${active_firewall[*]}"
	   fi
	 ]])

	if string.find(cmd_out, "PASS") then
		return true, "One firewall in use"
	else
		return false, "No firewalls or multiple firewalls in use"
	end
end

-------------------------------------------------------------------
-- REGISTRATION
-------------------------------------------------------------------

register_check({
	id = "BSD-123",
	name = "Is NetBSD",
	description = "Checks if the target is NetBSD",
	severity = "Info",
	run = is_bsd,
})

register_check({
	id = "UBU-111",
	name = "ufw is Installed",
	description = "Checks if the ufw firewall is installed",
	severity = "Medium",
	run = has_ufw,
})

register_check({
	id = "UBU-101",
	name = "One FW Active",
	description = "Check to ensure that only one firewall is active",
	severity = "Medium",
	run = one_fw,
})
